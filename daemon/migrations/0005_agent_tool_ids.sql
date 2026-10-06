UPDATE agent_definitions
SET tools = (
    SELECT json_group_array(json_extract(value, '$.name'))
    FROM json_each(agent_definitions.tools)
)
WHERE json_valid(agent_definitions.tools)
  AND json_array_length(agent_definitions.tools) > 0
  AND json_type(agent_definitions.tools, '$[0]') = 'object';
