use super::*;

impl Daemon {
    pub(super) fn dispatch(&self, connection: &ConnectionId, text: &str) {
        let event = match CloudEvent::from_json(text) {
            Ok(event) => event,
            Err(error) => {
                let report =
                    self.encoder
                        .office_error(event::CODE_INVALID_EVENT, &error.to_string(), None);
                self.send_direct(connection, report);
                return;
            }
        };

        let command = match Command::from_event(&event) {
            Ok(command) => command,
            Err(error) => {
                let rejection = self.encoder.command_rejected(
                    &event.id,
                    event::CODE_UNKNOWN_COMMAND,
                    &error.to_string(),
                );
                self.send_direct(connection, rejection);
                return;
            }
        };

        match command {
            Command::WorkspaceCreate(create) => {
                self.create_workspace(connection, &event, &create.name)
            }
            Command::WorkspaceList(_) => self.list_workspaces(connection, &event),
            Command::WorkspaceDelete(delete) => {
                self.delete_workspace(connection, &event, &delete.slug)
            }
            Command::ProjectCreate(create) => self.create_project(connection, &event, create),
            Command::ProjectList(list) => self.list_projects(connection, &event, list),
            Command::ProjectDelete(delete) => self.delete_project(connection, &event, delete),
            Command::AgentDefinitionCreate(create) => {
                self.create_agent_definition(connection, &event, create)
            }
            Command::AgentDefinitionList(_) => self.list_agent_definitions(connection, &event),
            Command::AgentDefinitionDelete(delete) => {
                self.delete_agent_definition(connection, &event, delete)
            }
            Command::AgentInstanceCreate(create) => {
                self.link_agent_instance(connection, &event, create)
            }
            Command::AgentInstanceList(list) => self.list_agent_instances(connection, &event, list),
            Command::AgentInstanceDelete(delete) => {
                self.unlink_agent_instance(connection, &event, delete)
            }
            Command::ToolList(_) => self.list_tools(connection, &event),
            Command::ModelList(_) => self.list_models(connection, &event),
            Command::SecretList(_) => self.list_secrets(connection, &event),
            Command::SecretSet(set) => self.set_secret(connection, &event, set),
            Command::SubscriptionAdd(added) => {
                let mut inner = self.state();
                inner.broker.subscribe(
                    connection.clone(),
                    Filter {
                        subject_prefix: added.filter.subject_prefix,
                    },
                );
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);
            }
            Command::SubscriptionRemove(_) => {
                let mut inner = self.state();
                inner.broker.unsubscribe(connection);
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);
            }
            Command::SubscriptionResume(resume) => {
                let inner = self.state();
                match inner.broker.replay(&resume.subject, resume.last_sequence) {
                    Ok(stored) => {
                        let accepted = self
                            .encoder
                            .command_accepted(&event.id, Some(&resume.subject));
                        send_to(&inner, connection, accepted);
                        for stored_event in stored {
                            send_payload(&inner, connection, stored_event.payload);
                        }
                    }
                    Err(ReplayError::ResyncRequired { subject, oldest }) => {
                        let message = format!(
                            "cursor {} is older than retained sequence {oldest}",
                            resume.last_sequence
                        );
                        let report = self.encoder.office_error(
                            event::CODE_RESYNC_REQUIRED,
                            &message,
                            Some(&subject),
                        );
                        send_to(&inner, connection, report);
                    }
                }
            }
            Command::TaskSubmit(submit) => self.submit_task(connection, &event, submit),
            Command::ExecutionCancel(cancel) => {
                let mut inner = self.state();
                let execution_id = ExecutionId(cancel.execution_id);
                match inner.executions.cancel(&execution_id) {
                    Ok(events) => {
                        let accepted = self
                            .encoder
                            .command_accepted(&event.id, event.subject.as_deref());
                        send_to(&inner, connection, accepted);
                        for domain_event in events {
                            publish_domain_event(
                                &mut inner,
                                &self.encoder,
                                domain_event,
                                Some(&event.id),
                            );
                        }
                    }
                    Err(error) => {
                        self.reject_into(&inner, connection, &event.id, &error);
                    }
                }
            }
        }
    }

    fn create_workspace(&self, connection: &ConnectionId, event: &CloudEvent, name: &str) {
        let result = offload(|| self.workspaces.create(name));
        let mut inner = self.state();

        match result {
            Ok(workspace) => {
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);

                let subject = format!("workspace:{}", workspace.slug);
                let payload = event::WorkspaceCreated {
                    name: workspace.name,
                    slug: workspace.slug,
                    user: workspace.user,
                    home: workspace.home,
                };
                let created = self
                    .encoder
                    .workspace_created(&subject, payload, Some(&event.id));
                publish(&mut inner, &subject, created);
            }
            Err(error) => {
                self.reject_into(&inner, connection, &event.id, &error);
            }
        }
    }

    fn list_workspaces(&self, connection: &ConnectionId, event: &CloudEvent) {
        let result = self.workspaces.list();
        let inner = self.state();

        match result {
            Ok(records) => {
                let payload = event::WorkspaceListed {
                    workspaces: records
                        .into_iter()
                        .map(|record| event::WorkspaceSummary {
                            name: record.name,
                            slug: record.slug,
                            user: record.user,
                            home: record.home,
                            created_at: record.created_at,
                            status: record.status,
                        })
                        .collect(),
                };
                let listed = self
                    .encoder
                    .workspace_listed(event.subject.as_deref(), payload);
                send_to(&inner, connection, listed);
            }
            Err(error) => {
                self.reject_into(&inner, connection, &event.id, &error);
            }
        }
    }

    fn delete_workspace(&self, connection: &ConnectionId, event: &CloudEvent, slug: &str) {
        match self.agents.list_instances(Some(slug), None) {
            Ok(instances) if !instances.is_empty() => {
                let inner = self.state();
                self.reject_into(
                    &inner,
                    connection,
                    &event.id,
                    &format!(
                        "workspace '{slug}' still has {} linked agent instance(s)",
                        instances.len()
                    ),
                );
                return;
            }
            Ok(_) => {}
            Err(error) => {
                self.reject(connection, &event.id, &error);
                return;
            }
        }

        match self.projects.list(Some(slug)) {
            Ok(projects) if !projects.is_empty() => {
                let inner = self.state();
                self.reject_into(
                    &inner,
                    connection,
                    &event.id,
                    &format!("workspace '{slug}' still has {} project(s)", projects.len()),
                );
                return;
            }
            Ok(_) => {}
            Err(error) => {
                self.reject(connection, &event.id, &error);
                return;
            }
        }

        let result = offload(|| self.workspaces.delete(slug));
        let mut inner = self.state();

        match result {
            Ok(workspace) => {
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);

                let subject = format!("workspace:{}", workspace.slug);
                let payload = event::WorkspaceDeleted {
                    slug: workspace.slug,
                };
                let deleted = self
                    .encoder
                    .workspace_deleted(&subject, payload, Some(&event.id));
                publish(&mut inner, &subject, deleted);
            }
            Err(error) => {
                self.reject_into(&inner, connection, &event.id, &error);
            }
        }
    }

    fn create_project(&self, connection: &ConnectionId, event: &CloudEvent, create: ProjectCreate) {
        match self.workspaces.list() {
            Ok(workspaces) if !workspaces.iter().any(|item| item.slug == create.workspace) => {
                let inner = self.state();
                self.reject_into(
                    &inner,
                    connection,
                    &event.id,
                    &format!("workspace '{}' not found", create.workspace),
                );
                return;
            }
            Ok(_) => {}
            Err(error) => {
                self.reject(connection, &event.id, &error);
                return;
            }
        }

        let result = self.projects.create(&create.workspace, &create.name);
        let mut inner = self.state();

        match result {
            Ok(project) => {
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);

                let subject = project_subject(&project.workspace, project.id.as_str());
                let created =
                    self.encoder
                        .project_created(&subject, project_view(project), Some(&event.id));
                publish(&mut inner, &subject, created);
            }
            Err(error) => {
                self.reject_into(&inner, connection, &event.id, &error);
            }
        }
    }

    fn list_projects(&self, connection: &ConnectionId, event: &CloudEvent, list: ProjectList) {
        let result = self.projects.list(list.workspace.as_deref());
        let inner = self.state();

        match result {
            Ok(records) => {
                let listed = self.encoder.project_listed(
                    event.subject.as_deref(),
                    event::ProjectListed {
                        projects: records.into_iter().map(project_summary).collect(),
                    },
                );
                send_to(&inner, connection, listed);
            }
            Err(error) => {
                self.reject_into(&inner, connection, &event.id, &error);
            }
        }
    }

    fn delete_project(&self, connection: &ConnectionId, event: &CloudEvent, delete: ProjectDelete) {
        match self
            .agents
            .list_instances(Some(&delete.workspace), Some(&delete.id))
        {
            Ok(instances) if !instances.is_empty() => {
                let inner = self.state();
                self.reject_into(
                    &inner,
                    connection,
                    &event.id,
                    &format!(
                        "project '{}/{}' still has {} linked agent instance(s)",
                        delete.workspace,
                        delete.id,
                        instances.len()
                    ),
                );
                return;
            }
            Ok(_) => {}
            Err(error) => {
                self.reject(connection, &event.id, &error);
                return;
            }
        }

        let result = self.projects.delete(&delete.workspace, &delete.id);
        let mut inner = self.state();

        match result {
            Ok(project) => {
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);

                let subject = project_subject(&project.workspace, project.id.as_str());
                let deleted = self.encoder.project_deleted(
                    &subject,
                    event::ProjectDeleted {
                        workspace: project.workspace,
                        id: project.id.0,
                    },
                    Some(&event.id),
                );
                publish(&mut inner, &subject, deleted);
            }
            Err(error) => {
                self.reject_into(&inner, connection, &event.id, &error);
            }
        }
    }

    fn create_agent_definition(
        &self,
        connection: &ConnectionId,
        event: &CloudEvent,
        create: AgentDefinitionCreate,
    ) {
        let resolved = match self.tools.resolve(&create.tools) {
            Ok(tools) => tools,
            Err(error) => {
                self.reject(connection, &event.id, &error);
                return;
            }
        };

        let model = create.model.trim().to_owned();
        if self.models.get(&model).is_none() {
            let inner = self.state();
            self.reject_into(
                &inner,
                connection,
                &event.id,
                &format!("unknown model '{model}'"),
            );
            return;
        }

        let result = self.agents.define(DefineAgent {
            name: create.name,
            role: create.role,
            prompt: create.prompt,
            tools: create.tools,
            model,
        });
        let mut inner = self.state();

        match result {
            Ok(definition) => {
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);

                let subject = format!("agent:{}", definition.id.as_str());
                let created = self.encoder.agent_definition_created(
                    &subject,
                    definition_view(definition, &resolved),
                    Some(&event.id),
                );
                publish(&mut inner, &subject, created);
            }
            Err(error) => {
                self.reject_into(&inner, connection, &event.id, &error);
            }
        }
    }

    fn list_agent_definitions(&self, connection: &ConnectionId, event: &CloudEvent) {
        let result = self.agents.list_definitions();
        let inner = self.state();

        match result {
            Ok(definitions) => {
                let mut views = Vec::with_capacity(definitions.len());
                for definition in definitions {
                    let resolved = match self.tools.resolve(&definition.tools) {
                        Ok(tools) => tools,
                        Err(error) => {
                            self.reject_into(&inner, connection, &event.id, &error);
                            return;
                        }
                    };
                    views.push(definition_view(definition, &resolved));
                }
                let listed = self.encoder.agent_definition_listed(
                    event.subject.as_deref(),
                    event::AgentDefinitionListed { definitions: views },
                );
                send_to(&inner, connection, listed);
            }
            Err(error) => {
                self.reject_into(&inner, connection, &event.id, &error);
            }
        }
    }

    fn delete_agent_definition(
        &self,
        connection: &ConnectionId,
        event: &CloudEvent,
        delete: AgentDefinitionDelete,
    ) {
        let result = self.agents.delete_definition(&delete.id);
        let mut inner = self.state();

        match result {
            Ok(id) => {
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);

                let subject = format!("agent:{id}");
                let deleted = self.encoder.agent_definition_deleted(
                    &subject,
                    event::AgentDefinitionDeleted { id },
                    Some(&event.id),
                );
                publish(&mut inner, &subject, deleted);
            }
            Err(error) => {
                self.reject_into(&inner, connection, &event.id, &error);
            }
        }
    }

    fn link_agent_instance(
        &self,
        connection: &ConnectionId,
        event: &CloudEvent,
        create: AgentInstanceCreate,
    ) {
        match self.projects.exists(&create.workspace, &create.project) {
            Ok(true) => {}
            Ok(false) => {
                let inner = self.state();
                self.reject_into(
                    &inner,
                    connection,
                    &event.id,
                    &format!(
                        "project '{}/{}' not found",
                        create.workspace, create.project
                    ),
                );
                return;
            }
            Err(error) => {
                self.reject(connection, &event.id, &error);
                return;
            }
        }

        let result = self
            .agents
            .link(&create.agent, &create.workspace, &create.project);
        let mut inner = self.state();

        match result {
            Ok(instance) => {
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);

                let subject = format!("agent-instance:{}", instance.id.as_str());
                let created = self.encoder.agent_instance_created(
                    &subject,
                    instance_view(instance),
                    Some(&event.id),
                );
                publish(&mut inner, &subject, created);
            }
            Err(error) => {
                self.reject_into(&inner, connection, &event.id, &error);
            }
        }
    }

    fn list_agent_instances(
        &self,
        connection: &ConnectionId,
        event: &CloudEvent,
        list: AgentInstanceList,
    ) {
        let result = self
            .agents
            .list_instances(list.workspace.as_deref(), list.project.as_deref());
        let inner = self.state();

        match result {
            Ok(records) => {
                let listed = self.encoder.agent_instance_listed(
                    event.subject.as_deref(),
                    event::AgentInstanceListed {
                        instances: records.into_iter().map(instance_summary).collect(),
                    },
                );
                send_to(&inner, connection, listed);
            }
            Err(error) => {
                self.reject_into(&inner, connection, &event.id, &error);
            }
        }
    }

    fn unlink_agent_instance(
        &self,
        connection: &ConnectionId,
        event: &CloudEvent,
        delete: AgentInstanceDelete,
    ) {
        let result = self
            .agents
            .unlink(&delete.agent, &delete.workspace, &delete.project);
        let mut inner = self.state();

        match result {
            Ok(removed) => {
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);

                if let Some(instance_id) = removed {
                    let subject = format!("agent-instance:{instance_id}");
                    let deleted = self.encoder.agent_instance_deleted(
                        &subject,
                        event::AgentInstanceDeleted {
                            agent: delete.agent,
                            workspace: delete.workspace,
                            project: delete.project,
                        },
                    );
                    publish(&mut inner, &subject, deleted);
                }
            }
            Err(error) => {
                self.reject_into(&inner, connection, &event.id, &error);
            }
        }
    }

    fn list_tools(&self, connection: &ConnectionId, event: &CloudEvent) {
        let tools = self.tools.list();
        let inner = self.state();

        let listed = self.encoder.tool_listed(
            event.subject.as_deref(),
            event::ToolListed {
                tools: tools.iter().map(wire_tool).collect(),
            },
        );
        send_to(&inner, connection, listed);
    }

    fn list_models(&self, connection: &ConnectionId, event: &CloudEvent) {
        self.reload_secrets();
        let models: Vec<event::ModelView> = self
            .models
            .list()
            .into_iter()
            .map(|model| {
                let provider = self.models.provider(model.provider.as_str());
                let provider_kind = provider
                    .as_ref()
                    .map(|item| item.kind.as_str().to_owned())
                    .unwrap_or_default();
                let endpoint = provider
                    .as_ref()
                    .map(|item| item.endpoint.clone())
                    .unwrap_or_default();
                let credential_configured = provider
                    .as_ref()
                    .map(|item| self.secrets.resolve(&item.credential_ref).is_some())
                    .unwrap_or(false);
                event::ModelView {
                    id: model.id.0,
                    provider: model.provider.0,
                    provider_kind,
                    endpoint,
                    api_name: model.api_name,
                    display_name: model.display_name,
                    context_window: model.context_window,
                    max_output_tokens: model.max_output_tokens,
                    capabilities: event::CapabilitiesView {
                        tool_calling: model.capabilities.tool_calling,
                        vision: model.capabilities.vision,
                        reasoning: model.capabilities.reasoning,
                    },
                    credential_configured,
                }
            })
            .collect();

        let inner = self.state();
        let listed = self
            .encoder
            .model_listed(event.subject.as_deref(), event::ModelListed { models });
        send_to(&inner, connection, listed);
    }

    fn list_secrets(&self, connection: &ConnectionId, event: &CloudEvent) {
        self.reload_secrets();
        let references = self.secrets.references();
        let inner = self.state();

        let listed = self
            .encoder
            .secret_listed(event.subject.as_deref(), event::SecretListed { references });
        send_to(&inner, connection, listed);
    }

    fn set_secret(&self, connection: &ConnectionId, event: &CloudEvent, set: SecretSet) {
        let Some(model) = self.models.get(&set.model) else {
            let inner = self.state();
            self.reject_into(
                &inner,
                connection,
                &event.id,
                &format!("unknown model '{}'", set.model),
            );
            return;
        };
        let Some(provider) = self.models.provider(model.provider.as_str()) else {
            let inner = self.state();
            self.reject_into(
                &inner,
                connection,
                &event.id,
                &format!("model '{}' has no provider", set.model),
            );
            return;
        };

        match offload(|| self.secrets.set(&provider.credential_ref, &set.value)) {
            Ok(()) => {
                let mut inner = self.state();
                let accepted = self
                    .encoder
                    .command_accepted(&event.id, event.subject.as_deref());
                send_to(&inner, connection, accepted);

                let subject = format!("secret:{}", provider.credential_ref);
                let updated = self.encoder.secret_updated(
                    &subject,
                    event::SecretUpdated { model: set.model },
                    Some(&event.id),
                );
                publish(&mut inner, &subject, updated);
            }
            Err(error) => {
                self.reject(connection, &event.id, &error);
            }
        }
    }

    pub(super) fn reload_secrets(&self) {
        if let Err(error) = offload(|| self.secrets.reload()) {
            tracing::error!(error = %error, "could not reload secrets");
        }
    }
}
