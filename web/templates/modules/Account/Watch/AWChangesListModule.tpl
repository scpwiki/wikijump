{if isset($revisions)}

	{pager jsfunction="Wikijump.modules.AWChangesModule.listeners.updateList(event,#)" total=$pagerData.totalPages known=$pagerData.knownPages current=$pagerData.currentPage}


	{foreach from=$revisions item=revision}

		{assign var=page value=$revision->getPage()}
		{assign var=site value=$page->getSite()}
		<div class="changes-list-item">

			<table>
				<tr>
					<td class="site">
						<a href="{$HTTP_SCHEMA}://{$site->getDomain()}">{$site->getName()|escape}</a>
					</td>
					<td class="title">
						<a href="/{$page->getUnixname()}">{if $page->getTitle()|escape}{$page->getTitle()|escape}{else}{$page->getUnixName()|escape}{/if}</a>
					</td>
					<td class="flags">
						{if $revision->getFlagNew()}
					 		<span class="spantip" title="new page created">N</span>
					 	{/if}
					 	{if $revision->getFlagText()}
					 		<span class="spantip" title="content source text changed">S</span>
					 	{/if}
					 	{if $revision->getFlagTitle()}
					 		<span class="spantip" title="title changed">T</span>
					 	{/if}
					 	{if $revision->getFlagRename()}
					 		<span class="spantip" title="page renamed/moved">R</span>
					 	{/if}
					 	{if $revision->getFlagFile()}
					 		<span class="spantip" title="file/attachment action">F</span>
					 	{/if}
					 	{if $revision->getFlagMeta()}
					 		<span class="spantip" title="meta data changed">M</span>
					 	{/if}
					</td>
					<td  class="mod-date">
						<span class="odate">{$revision->getDateLastEdited()->getTimestamp()}|%e %b %Y - %H:%M:%S|agohover</span>
					</td>
					<td class="revision-no">
						({if $revision->getRevisionNumber() == 0}new{else}rev. {$revision->getRevisionNumber()}{/if})
					</td>
				</tr>
			</table>

			{if $revision->getComments()}
				<div class="comments">
					{$revision->getComments()}
				</div>
			{/if}


		</div>
	{/foreach}

	{if $revisionsCount > 10}
		{pager jsfunction="Wikijump.modules.UserChangesModule.listeners.updateList" total=$pagerData.totalPages known=$pagerData.knownPages current=$pagerData.currentPage}
	{/if}
{else}
	Sorry, no revisions matching your criteria.
{/if}
