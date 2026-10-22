## {{ title }}

{% for line in compiler_lines %}- {{ line }}
{% endfor %}- contracts: {{ contracts }}

| outcome | count | share | meaning |
|---|---|---|---|
{% for row in outcome_rows %}| {{ row|join(" | ") }} |
{% endfor -%}
{% for census in censuses %}
### {{ census.title }} ({{ census.total }})

kinds: {{ census.kinds }}

| count | kind | signature | examples |
|---|---|---|---|
{% for row in census.rows %}| {{ row|join(" | ") }} |
{% endfor -%}
{% if let Some(more) = census.more %}
{{ more }}
{% endif -%}
{% endfor -%}
{% if let Some(timeouts) = timeouts %}
### Timeouts ({{ timeouts.count }})

{{ timeouts.items }}
{% endif -%}
