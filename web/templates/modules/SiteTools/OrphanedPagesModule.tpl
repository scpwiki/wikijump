<h1>List of orphaned pages</h1>

<p>
	Below is the list of pages that do not have any incoming links from other pages -
	at least internal links produced by syntax <tt>[[[page-name]]]</tt>. If a page is listed here
	it should not mean anything wrong because there might be special pages that do not
	(and <u>should not</u> as e.g. some forum pages) have incoming
	links. But it is recommended to check this list from time to time.
</p>

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
