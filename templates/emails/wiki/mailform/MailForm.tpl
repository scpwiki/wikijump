Form data:

{foreach from=$fields item=field}
{$field.title}:
{$field.value}

{/foreach}