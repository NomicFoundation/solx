### 🧪 Integration tests — {% if full_matrix %}full matrix{% else %}standard{% endif %} · PR vs `main`

{% match output -%}
{% when OutputVerdict::NoData -%}
⚪ **No output data** — no size or gated-gas comparisons were collected.
{% when OutputVerdict::Preserving with { size_cells, gated_gas_cells, gas_label } -%}
✅ **Output-preserving** — {% if *size_cells > 0 %}bytecode size identical ({{ size_cells|commas }} comparisons){% endif %}{% if *size_cells > 0 && *gated_gas_cells > 0 %}, {% endif %}{% if *gated_gas_cells > 0 %}{{ gas_label }} gas identical ({{ gated_gas_cells|commas }}){% endif %}.
{% when OutputVerdict::Changed with { size, gas } -%}
⚠️ **Output changed** — {% if let Some(size) = size %}{{ size.diffs|commas }} of {{ size.cells|commas }} size comparisons differ ({{ "{:+}"|format(size.delta_bytes) }} B total){% endif %}{% if size.is_some() && gas.is_some() %}; {% endif %}{% if let Some(gas) = gas %}{{ gas.diffs|commas }} of {{ gas.cells|commas }} {{ gas.label }} gas comparisons differ{% endif %}. If this PR is meant to be output-preserving, investigate before merging.
{% endmatch -%}
{% match failures -%}
{% when FailureVerdict::NoData -%}
⚪ **No failure data** — no PR run had a `main` counterpart to compare against.
{% when FailureVerdict::Clean with { pre_existing } -%}
{% if pre_existing.is_empty() -%}
✅ **No new failures**.
{% else -%}
✅ **No new failures** — {% for (label, count) in pre_existing %}{% if !loop.first %} / {% endif %}{{ label }}'s {{ count|commas_usize }}{% endfor %} failures already present on `main`.
{% endif -%}
{% when FailureVerdict::Regressed with { suites } -%}
❌ **New failures** — {% for suite in suites %}{% if !loop.first %}; {% endif %}{{ suite.label }}: {% if suite.new_build > 0 %}+{{ suite.new_build|commas_usize }} build{% endif %}{% if suite.new_build > 0 && suite.new_test > 0 %}, {% endif %}{% if suite.new_test > 0 %}+{{ suite.new_test|commas_usize }} test{% endif %}{% endfor %}.
{% endmatch -%}
{% for issue in issues -%}
{% match issue -%}
{% when HealthIssue::SuiteErrored with { label } -%}
❌ **Suite errored** — {{ label }} produced no usable report.
{% when HealthIssue::EmptySuite with { label } -%}
❌ **Suite empty** — {{ label }}'s report contains no runs.
{% when HealthIssue::UnrecognizedToolchains with { label } -%}
❌ **Harness error** — {{ label }}: benchmark data matched no recognized toolchain naming.
{% when HealthIssue::UnrecognizedRuns with { label, modes } -%}
❌ **Harness error** — {{ label }}: runs matched no declared toolchain: {% for mode in modes %}{% if loop.index0 < MAX_LISTED %}{% if !loop.first %}, {% endif %}`{{ mode }}`{% endif %}{% endfor %}{% if modes.len() > MAX_LISTED %} (+{{ modes.len() - MAX_LISTED }} more){% endif %}.
{% when HealthIssue::Unbaselined with { label, runs, failures } -%}
{% endmatch -%}
{% endfor -%}
{% if !unbaselined.is_empty() -%}
⚠️ **No baseline** — {% for part in unbaselined %}{% if !loop.first %}; {% endif %}{{ part }}{% endfor %} have no `main` counterpart; their failures are not compared.
{% endif %}
| Suite | New failures | Size Δ | Gas Δ | Report |
|---|---|---|---|---|
{% for s in stats %}
{%- if !s.available %}| {{ s.label }} | ❌ no report — suite errored | — | — | {{ s.report_cell() }} |
{% else if s.is_empty_report() %}| {{ s.label }} | ❌ empty report | — | — | {{ s.report_cell() }} |
{% else if s.classification_failed() %}| {{ s.label }} | ❌ unrecognized toolchain naming | — | — | {{ s.report_cell() }} |
{% else %}| {{ s.suite_cell() }} | {{ s.failures_cell() }} | {{ s.size.cell(true) }} | {{ s.gas_cell() }} | {{ s.report_cell() }} |
{% endif %}
{%- endfor -%}
{% if has_new_failures %}
**New failures (PR vs `main`):**

{% for s in stats %}
{%- for regression in s.failure_regressions %}
{%- if loop.index0 < MAX_LISTED %}- {{ s.label }}: `{{ regression.label }}` [{{ regression.mode }}] {{ regression.kind }} failures {{ regression.main }} → {{ regression.pr }}
{% endif %}
{%- endfor %}
{%- if s.failure_regressions.len() > MAX_LISTED %}- +{{ s.failure_regressions.len() - MAX_LISTED }} more — see {{ s.report_file }}
{% endif %}
{%- endfor %}
{%- endif %}
{%- for section in mover_sections %}
**{{ section.suite_label }} — {{ section.title }}:**

{% for m in section.movers %}
{%- if loop.index0 < MAX_LISTED %}- `{{ m.label }}` [{{ m.mode }}] {{ m.main|commas }} → {{ m.pr|commas }}{{ section.unit }}{{ m.pr|rel_suffix(m.main) }}
{% endif %}
{%- endfor %}
{%- if section.movers.len() > MAX_LISTED %}- +{{ section.movers.len() - MAX_LISTED }} more — full list in {{ section.report_file }}
{% endif %}
{%- endfor %}
{%- if let Some(compile) = compile %}
**Compile time** — wall-clock tripwire, positive = PR slower (authoritative Δ in `ci:compile-benchmark`)

| Suite |{% for pipeline in compile.pipelines %} {{ pipeline }} (agg / median) |{% endfor %}
|---|{% for pipeline in compile.pipelines %}---|{% endfor %}
{% for row in compile.rows %}
{%- for cell in row %}{% if loop.first %}| {{ cell }} |{% else %} {{ cell }} |{% endif %}{% endfor %}
{% endfor %}
{%- if compile.within_noise %}
_Within noise — no suite ≥ {{ COMPILE_TIME_SUITE_THRESHOLD_PERCENT|floor_u64 }}%, no project ≥ {{ COMPILE_TIME_PROJECT_THRESHOLD_PERCENT|floor_u64 }}%._
{% endif %}
{%- if let Some(outliers) = compile.outliers %}
{{ outliers }}
{% endif %}
{%- endif %}
{%- if !baseline_rows.is_empty() %}
**Bytecode size — PR vs baselines** (positive = PR larger; contracts built by both only)

| Suite | Pipeline | vs solc | vs released solx |
|---|---|---|---|
{% for row in baseline_rows %}
{%- for cell in row %}{% if loop.first %}| {{ cell }} |{% else %} {{ cell }} |{% endif %}{% endfor %}
{% endfor %}
{%- endif %}
