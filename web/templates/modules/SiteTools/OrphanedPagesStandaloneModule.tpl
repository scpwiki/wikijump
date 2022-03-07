<h1>List of orphaned pages</h1>

{if $pages}
	{foreach from=$pages item=page}
		<a href="/{$page->slug}">{$page->title|escape}</a> <span style="color: #999">({$page->slug})</span>
		<br/>
	{/foreach}
{else}
	<p>
		No orphaned pages found.
	</p>
{/if}
