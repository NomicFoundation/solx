### 🧪 Integration tests — {% if full_matrix %}full matrix{% else %}standard{% endif %} · PR vs `main`

{{ output_line }}
{{ failures_line }}
{% for line in health_lines %}{{ line }}
{% endfor -%}
{% for line in warn_lines %}{{ line }}
{% endfor %}
| Suite | New failures | Size Δ | Gas Δ | Report |
|---|---|---|---|---|
{% for row in suite_rows %}| {{ row.suite }} | {{ row.failures }} | {{ row.size }} | {{ row.gas }} | {{ row.report }} |
{% endfor -%}
{% if !new_failure_bullets.is_empty() %}
**New failures (PR vs `main`):**

{% for bullet in new_failure_bullets %}- {{ bullet }}
{% endfor -%}
{% endif -%}
{% for section in mover_sections %}
**{{ section.heading }}:**

{% for bullet in section.bullets %}- {{ bullet }}
{% endfor -%}
{% endfor -%}
{% if let Some(compile) = compile %}
**Compile time** — wall-clock tripwire, positive = PR slower (authoritative Δ in `ci:compile-benchmark`)

| Suite |{% for pipeline in compile.pipelines %} {{ pipeline }} (agg / median) |{% endfor %}
|---|{% for pipeline in compile.pipelines %}---|{% endfor %}
{% for row in compile.rows %}| {{ row|join(" | ") }} |
{% endfor -%}
{% if let Some(line) = compile.conclusion_line %}
{{ line }}
{% endif -%}
{% if let Some(line) = compile.outliers_line %}
{{ line }}
{% endif -%}
{% endif -%}
{% if !baseline_rows.is_empty() %}
**Bytecode size — PR vs baselines** (positive = PR larger; contracts built by both only)

| Suite | Pipeline | vs solc | vs released solx |
|---|---|---|---|
{% for row in baseline_rows %}| {{ row|join(" | ") }} |
{% endfor -%}
{% endif -%}
