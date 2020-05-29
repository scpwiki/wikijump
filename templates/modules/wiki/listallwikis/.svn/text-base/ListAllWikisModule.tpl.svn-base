<ul>
	{foreach from=$sites item=site}
		<li>
			<a href="http://{$site->getDomain()}">{$site->getName()|escape}</a>
			{if $site->getSubtitle()} - {$site->getSubtitle()|escape}{/if}
		</li>
	{/foreach}
</ul>

