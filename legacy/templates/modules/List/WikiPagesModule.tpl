{if $pages}
<div class="pages-list">
	{foreach	 from=$pages item=page}
		<div class="pages-list-item">
			{if $details}
				<table>
					<tr>
						<td class="title">
							<a href="/{$page->getUnixname()}">{if $page->title|escape}{$page->title|escape}{else}{$page->slug|escape}{/if}</a>
						</td>
						<td class="last-mod-by">
							{printuser user=$page->revision_user_id}
						</td>
						<td class="revision-no">
							({if $page->getRevisionNumber() == 0}new{else}rev. {$page->getRevisionNumber()}{/if})
						</td>
						<td class="last-mod-date">
							<span class="odate">{$page->getDateLastEdited()->getTimestamp()}|%e %b %Y - %H:%M|agohover</span>
						</td>
					</tr>
				</table>
			{else}
				<div class="title">
					<a href="/{$page->getUnixname()}">{if $page->title|escape}{$page->title|escape}{else}{$page->slug|escape}{/if}</a>
				</div>
			{/if}
		</div>
	{/foreach}
</div>
{else}
no pages.
{/if}
