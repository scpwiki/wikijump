{if isset($raw)}{$source}{else}
<h1>{t}Page source{/t}</h1>

<div class="page-source">
	{$source|escape|semipre}
</div>
{/if}
