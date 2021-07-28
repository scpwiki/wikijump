{if $raw}{$source}{else}
<h1>{t}Page source{/t}</h1>

<pre><textarea class="page-source" readonly>{$source|escape}</textarea></pre>
{/if}
