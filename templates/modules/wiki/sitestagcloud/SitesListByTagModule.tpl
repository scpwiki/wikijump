{if $title !== null}
	<h2>{$title|escape}</h2>
{else}
	<h2>{t}Sites list for the tag{/t} <em>{$tag|escape}</em></h2>
{/if}

<div class="sites-list">
	{foreach from=$sites item=site}
		<div class="site-list-item">
			<div class="name">
				<a href="http://{$site->getDomain()}">{$site->getName()|escape}</a>
			</div>
			{if $site->getSubtitle()}
				<div class="subtitle">
					{$site->getSubtitle()|escape}
				</div>
			{/if}
			{if $site->getDescription()}
				<div class="description">
					{$site->getDescription()|escape}
				</div>
			{/if}
		</div>
	{/foreach}
</div>