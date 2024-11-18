# Current System Info

{{system.info()}}

# Hints

{{synopsis.hints}}

# Relevant Files

{% for file in system.active_files %}
{{file.path}}
```{{file.language}}
{{file.content}}
```

{% endfor %}

# Summary

{{synopsis.current_summary}}

# Suggested Plan

{{synopsis.current_plan}}