<h1>{t}Diff changes{/t}</h1>
<p>
	<a href="javascript:;" class="button" onclick="Wikijump.modules.PageEditModule.listeners.closeDiffView(event)">{t}close diff view{/t}</a>
</p>
<pre><div class="inline-diff page-source">{if $diff}{$diff}{else}{t}The documents are identical - no changes.{/t}{/if}</div></pre>
