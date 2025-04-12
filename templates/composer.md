
# {{ project.github_organization }}

{% for repo in project.repositories %}
## {{ repo.name }}
`clone_url`: {{ repo.clone_url }}

{% for source in repo.sources %}
### {{ source.relative_path }}
`path`: {{ source.path }}
`relative_path`: {{ source.relative_path }}
`format`: {{ source.format }}
`size`: {{ source.size }}   

{% match source.content %}
{% when Some with (val)  %}
``````
{{ val }}
``````
{% when None %}
{% endmatch %}

{% endfor %}
{% endfor %}
