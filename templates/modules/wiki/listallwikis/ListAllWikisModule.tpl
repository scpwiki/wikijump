<ul>
	{foreach from=$sites item=site}
		<li>
			<a href="{$HTTP_SCHEMA}{$site->getDomain()}">{$site->getName()|escape}</a>
			{if $site->getSubtitle()} - {$site->getSubtitle()|escape}{/if}
		</li>
	{/foreach}
</ul>

