<div class="recent-w-page-revisions" id="recent-w-page-revisions">
{*
	{if $revisions}
		{foreach from=$revisions item=revision}
			{assign var=page value=$revision->getPage()}
			{assign var=site value=$page->getSite()}
			<div class="list-item">
				<div class="title">
					<a href="http://{$site->getDomain()|escape}/{$page->getUnixName()}">{$page->getTitleOrUnixname()|escape}</a>
				</div>
				<div class="in-site">
					in Site: <a href="http://{$site->getDomain()}">{$site->getName()|escape}</a>, <br/><span class="odate">{$revision->getDateLastEdited()->getTimestamp()}|%O {t}ago{/t}</span>
				</div>
			</div>
		{/foreach}
	{/if}
*}
	{if $pages}
		{foreach from=$pages item=page}
			{assign var=site value=$page->getSite()}
			<div class="list-item">
				<div class="title">
					<a href="{$HTTP_SCHEMA}://{$site->getDomain()|escape}/{$page->getUnixName()}">{$page->getTitleOrUnixname()|escape}</a>
				</div>
				<div class="in-site">
					{t}site{/t}: <a href="{$HTTP_SCHEMA}://{$site->getDomain()}">{$site->getName()|escape}</a>, <br/><span class="odate">{$page->getDateLastEdited()->getTimestamp()}|%O {t}ago{/t}</span>
				</div>
			</div>
		{/foreach}
	{/if}

</div>
