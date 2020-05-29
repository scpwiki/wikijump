
{if $pagesCount >0}
	<table>
		{*<tr>
			<th>
				site
			</th>
			<th>
				page name
			</th>
			<th> </th>
		</tr>*}
		{foreach from=$pages item=page}
		{assign var=site value=$page->getSite()}
			<tr>
				<td>
					site: <a href="http://{$site->getDomain()}">{$site->getName()|escape}</a>
				</td>
				<td>
					| <a href="http://{$site->getDomain()}/{$page->getUnixName()}">{$page->getTitle()|escape}</a>
				</td>
				<td>
					| <a href="javascript:;" onclick="WIKIDOT.modules.AWChangesModule.listeners.removeWatchedPage(event, {$page->getPageId()})">remove</a>
				</td>
			</tr>
		{/foreach}
	</table>
{else}
	You are watching no pages now.
{/if}

<p>
	To watch a new page simply open this page in the browser, click on "history" 
	option and "add to watched".
</p>