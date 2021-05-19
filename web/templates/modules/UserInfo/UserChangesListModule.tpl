{if isset($revisions)}

	{pager jsfunction="Wikijump.modules.UserChangesModule.listeners.updateList" total=$pagerData.totalPages known=$pagerData.knownPages current=$pagerData.currentPage}


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
						<a href="{$HTTP_SCHEMA}://{$site->getDomain()|escape}/{$page->getUnixname()}">{if $page->getTitle()|escape}{$page->getTitle()|escape}{else}{$page->getUnixName()|escape}{/if}</a>
					</td>
					<td class="flags">
						{if $revision->getFlagNew()}
					 		<span class="spantip" title="{t}new page created{/t}">N</span>
					 	{/if}
					 	{if $revision->getFlagText()}
					 		<span class="spantip" title="{t}content source text changed{/t}">S</span>
					 	{/if}
					 	{if $revision->getFlagTitle()}
					 		<span class="spantip" title="{t}title changed{/t}">T</span>
					 	{/if}
					 	{if $revision->getFlagRename()}
					 		<span class="spantip" title="{t}page renamed/moved{/t}">R</span>
					 	{/if}
					 	{if $revision->getFlagFile()}
					 		<span class="spantip" title="{t}file/attachment action{/t}">F</span>
					 	{/if}
					 	{if $revision->getFlagMeta()}
					 		<span class="spantip" title="{t}meta data changed{/t}">M</span>
					 	{/if}
					</td>
					<td  class="mod-date">
						<span class="odate">{$revision->getDateLastEdited()->getTimestamp()}|%e %b %Y - %H:%M:%S|agohover</span>
					</td>
					<td class="revision-no">
						({if $revision->getRevisionNumber() == 0}{t}new{/t}{else}{t}rev.{/t} {$revision->getRevisionNumber()}{/if})
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
	{t}Sorry, no revisions matching your criteria.{/t}
{/if}
