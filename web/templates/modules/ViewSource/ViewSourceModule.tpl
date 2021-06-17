{if $raw}{$source}{else}
<h1>{t}Page source{/t}</h1>

<textarea class="page-source" readonly>
	{$source|escape|semipre}
</textarea>
{/if}
