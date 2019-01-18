
{if $threadsCount >0}
	<table>
		{*<tr>
			<th>
				site
			</th>
			<th>
				thread name
			</th>
			<th> </th>
		</tr>*}
		{foreach from=$threads item=thread}
		{assign var=site value=$thread->getSite()}
			<tr>
				<td>
					site: <a href="http://{$site->getDomain()}">{$site->getName()|escape}</a>
				</td>
				<td>
					| <a href="http://{$site->getDomain()}/forum/t-{$thread->getThreadId()}/{$thread->getUnixifiedTitle()|escape}">{$thread->getTitle()|escape}</a>
				</td>
				<td>
					| <a href="javascript:;" onclick="WIKIDOT.modules.AWForumModule.listeners.removeWatchedThread(event, {$thread->getThreadId()})">remove</a>
				</td>
			</tr>
		{/foreach}
	</table>
{else}
	You are watching no threads now.
{/if}

<p>
	To watch a new forum thread simply open this thread in the browser, click on "+ more options" 
	link under thread summary and click "add to watched".
</p>