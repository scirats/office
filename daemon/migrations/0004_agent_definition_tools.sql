ALTER TABLE agent_definitions RENAME COLUMN skills TO tools;
ALTER TABLE agent_definitions DROP COLUMN runtime;
ALTER TABLE agent_definitions DROP COLUMN policy;
