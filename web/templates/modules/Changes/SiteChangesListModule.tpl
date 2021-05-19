{if isset($revisions)}

	{pager jsfunction="Wikijump.modules.SiteChangesModule.listeners.updateList" total=$pagerData.totalPages known=$pagerData.knownPages current=$pagerData.currentPage}


	{foreach from=$revisions item=revision}

		{assign var=page value=$revision->getPage()}
		<div class="changes-list-item">

			<table>
				<tr>
					<td class="title">
						<a href="/{$page->getUnixname()}">{if $page->getTitle()|escape}{$page->getTitle()|escape}{else}{$page->getUnixName()|escape}{/if}</a>
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
						({if $revision->getRevisionNumber() == 0}{t}new{/t}{else}{t}rev{/t}. {$revision->getRevisionNumber()}{/if})
					</td>
					<td class="mod-by">
						{printuser user=$revision->getUserOrString()}
					</td>
				</tr>
			</table>

			{if $revision->getComments()}
				<div class="comments">
					{$revision->getComments()}
				</div>
			{/if}

			{*

			{assign var=page value=$revision->getPage()}
			<div class="mod-by">
				{printuser user=$revision->getUserOrString()}
			</div>
			<div class="revision-no">
					({if $revision->getRevisionNumber() == 0}new{else}rev. {$revision->getRevisionNumber()}{/if})
				</div>
			<div class="mod-date">
				<span class="odate">{$revision->getDateLastEdited()->getTimestamp()}|%e %b %Y - %H:%M:%S %Z|agohover</span>
			</div>
			<div class="flags">
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
			</div>


			<div class="title">
				<a href="/{$page->getUnixname()}">{if $page->getTitle()|escape}{$page->getTitle()|escape}{else}{$page->getUnixName()|escape}{/if}</a>
			</div>
			{if $revision->getComments()}
				<div class="comments">
					{$revision->getComments()}
				</div>
			{/if}
			*}
		</div>
	{/foreach}

	{if $revisionsCount > 10}
		{pager jsfunction="Wikijump.modules.SiteChangesModule.listeners.updateList" total=$pagerData.totalPages known=$pagerData.knownPages current=$pagerData.currentPage}
	{/if}
{else}
	{t}Sorry, no revisions matching your criteria.{/t}
{/if}
