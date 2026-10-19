use super::*;

impl Daemon {
    pub(super) fn submit_task(
        &self,
        connection: &ConnectionId,
        event: &CloudEvent,
        submit: TaskSubmit,
    ) {
        let agent_id = agent::slugify(&submit.agent);
        let prepared = match self.prepare_run(&agent_id, &submit.input, &submit.project) {
            Ok(prepared) => prepared,
            Err(error) => {
                self.reject_with(connection, &event.id, error.code(), &error);
                return;
            }
        };

        let mut inner = self.state();
        let domain_events = inner.executions.accept_task();
        let accepted = self
            .encoder
            .command_accepted(&event.id, event.subject.as_deref());
        send_to(&inner, connection, accepted);

        let mut execution_id = None;
        for domain_event in domain_events {
            if let DomainEvent::TaskAccepted {
                execution_id: id, ..
            } = &domain_event
            {
                execution_id = Some(id.clone());
            }
            publish_domain_event(&mut inner, &self.encoder, domain_event, Some(&event.id));
        }

        if let Some(execution_id) = execution_id {
            if let Ok(events) = inner
                .executions
                .transition(&execution_id, ExecutionState::Preparing)
            {
                for domain_event in events {
                    publish_domain_event(&mut inner, &self.encoder, domain_event, Some(&event.id));
                }
            }

            let daemon = self.clone();
            tokio::spawn(async move { daemon.run_execution(execution_id, prepared).await });
        }
    }

    fn prepare_run(
        &self,
        agent_id: &str,
        input: &str,
        project: &str,
    ) -> Result<PreparedRun, RunResolveError> {
        let definition = self
            .agents
            .list_definitions()?
            .into_iter()
            .find(|definition| definition.id.as_str() == agent_id)
            .ok_or_else(|| RunResolveError::UnknownAgent(agent_id.to_owned()))?;

        let model = self
            .models
            .get(&definition.model)
            .ok_or_else(|| RunResolveError::UnknownModel(definition.model.clone()))?;
        let provider = self
            .models
            .provider(model.provider.as_str())
            .ok_or_else(|| RunResolveError::UnknownProvider(model.provider.0.clone()))?;
        let secret = self
            .secrets
            .resolve(&provider.credential_ref)
            .ok_or_else(|| RunResolveError::MissingCredential(provider.id.0.clone()))?;

        let mut tools = Vec::with_capacity(definition.tools.len());
        for id in &definition.tools {
            let tool = self
                .tools
                .get(id)
                .ok_or_else(|| RunResolveError::UnknownTool(id.clone()))?;
            tools.push(ChatTool {
                name: tool.name,
                description: tool.description,
                parameters: tool.parameters,
            });
        }

        Ok(PreparedRun {
            endpoint: provider.endpoint,
            api_key: secret.expose().to_owned(),
            model: model.api_name,
            system: definition.prompt,
            input: input.to_owned(),
            tools,
            scope: self.resolve_scope(agent_id, project)?,
        })
    }

    fn resolve_scope(&self, agent_id: &str, project: &str) -> Result<RunScope, RunResolveError> {
        if project.trim().is_empty() {
            return Ok(RunScope::Agent);
        }

        let instance = self
            .agents
            .list_instances(None, Some(project))?
            .into_iter()
            .find(|instance| instance.agent.as_str() == agent_id)
            .ok_or_else(|| RunResolveError::UnknownInstance {
                agent: agent_id.to_owned(),
                project: project.to_owned(),
            })?;
        let home = self
            .workspaces
            .list()?
            .into_iter()
            .find(|workspace| workspace.slug == instance.workspace)
            .map(|workspace| workspace.home)
            .ok_or_else(|| RunResolveError::UnknownWorkspace(instance.workspace.clone()))?;

        Ok(RunScope::Instance {
            workspace: instance.workspace,
            home,
            project: project.to_owned(),
        })
    }

    async fn run_execution(self, execution_id: ExecutionId, prepared: PreparedRun) {
        self.transition(&execution_id, ExecutionState::Ready);
        self.transition(&execution_id, ExecutionState::Running);

        let chat = Arc::new(InferenceChat::new(
            self.chat.clone(),
            prepared.endpoint,
            prepared.api_key,
            prepared.model,
        ));
        let runner = RunnerService::new(chat, self.tool_runner.clone());
        let request = RunRequest {
            system: prepared.system,
            input: prepared.input,
            tools: prepared.tools,
            context: RunContext {
                execution_id: execution_id.0.clone(),
                scope: prepared.scope,
            },
            max_steps: self.policy.max_steps,
        };

        let mut emit = |event: RunEvent| match event {
            RunEvent::Text(text) => self.log(&execution_id, LogStream::Stdout, text),
            RunEvent::ToolCall(call) => self.log(
                &execution_id,
                LogStream::Tool,
                format!("{} {}", call.name, call.arguments),
            ),
            RunEvent::ToolResult { output, .. } => {
                self.log(&execution_id, LogStream::ToolResult, output)
            }
        };

        match runner.run(request, &mut emit).await {
            Ok(()) => {
                self.transition(&execution_id, ExecutionState::Finishing);
                self.transition(&execution_id, ExecutionState::Succeeded);
            }
            Err(error) => {
                tracing::warn!(
                    target: "office.execution",
                    execution = %execution_id.as_str(),
                    error = %error,
                    "execution failed"
                );
                self.log(&execution_id, LogStream::Stderr, error.to_string());
                self.transition(&execution_id, ExecutionState::Failed);
            }
        }

        self.cleanup_execution(&execution_id);
    }

    fn cleanup_execution(&self, execution_id: &ExecutionId) {
        let dir = self.policy.execution_dir(execution_id.as_str());
        if dir.exists()
            && let Err(error) = std::fs::remove_dir_all(&dir)
        {
            tracing::warn!(
                target: "office.execution",
                directory = %dir.display(),
                error = %error,
                "could not clean execution directory"
            );
        }
    }

    fn transition(&self, execution_id: &ExecutionId, to: ExecutionState) {
        let mut inner = self.state();
        if let Ok(events) = inner.executions.transition(execution_id, to) {
            for domain_event in events {
                publish_domain_event(&mut inner, &self.encoder, domain_event, None);
            }
        }
    }

    fn log(&self, execution_id: &ExecutionId, stream: LogStream, line: impl Into<String>) {
        let mut inner = self.state();
        if let Ok(events) = inner.executions.log(execution_id, stream, line) {
            for domain_event in events {
                publish_domain_event(&mut inner, &self.encoder, domain_event, None);
            }
        }
    }
}
