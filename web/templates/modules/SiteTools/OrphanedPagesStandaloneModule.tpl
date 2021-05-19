<h1>List of orphaned pages</h1>

{if isset($pages)}
	{foreach from=$pages item=page}
		<a href="/{$page->getUnixName()}">{$page->getTitleOrUnixName()|escape}</a> <span style="color: #999">({$page->getUnixName()})</span>
		<br/>
	{/foreach}
{else}
	<p>
		No orphaned pages found.
	</p>
{/if}
