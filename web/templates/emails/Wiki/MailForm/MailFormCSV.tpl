# {foreach from=$fields item=field name="names"}{$field.name}{if !$smarty.foreach.names.last},{/if}{/foreach}


{foreach from=$fields item=field name="values"}{$field.value}{if !$smarty.foreach.values.last},{/if}{/foreach}
