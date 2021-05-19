{if isset($pages)}
<div class="pages-list">
	{foreach	 from=$pages item=page}
		<div class="pages-list-item">
			{if isset($details)}
				<table>
					<tr>
						<td class="title">
							<a href="/{$page->getUnixname()}">{if $page->getTitle()|escape}{$page->getTitle()|escape}{else}{$page->getUnixName()|escape}{/if}</a>
						</td>
						<td class="last-mod-by">
							{printuser user=$page->getLastEditUserOrString()}
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
					<a href="/{$page->getUnixname()}">{if $page->getTitle()|escape}{$page->getTitle()|escape}{else}{$page->getUnixName()|escape}{/if}</a>
				</div>
			{/if}
			{if isset($preview)}
				<div class="preview">
					{$page->getPreview()}
				</div>
			{/if}
		</div>

	{*	<div class="pages-list-item">
			{if isset($details)}

				<div class="last-mod-by">
					{printuser user=$page->getLastEditUserOrString()}
				</div>
				<div class="revision-no">
					({if $page->getRevisionNumber() == 0}new{else}rev. {$page->getRevisionNumber()}{/if})
				</div>
				<div class="last-mod-date">
					<span class="odate">{$page->getDateLastEdited()->getTimestamp()}|%e %b %Y - %H:%M|agohover</span>
				</div>
			{/if}
			<div class="title">
				<a href="/{$page->getUnixname()}">{if $page->getTitle()|escape}{$page->getTitle()|escape}{else}{$page->getUnixName()|escape}{/if}</a>
			</div>
			{if isset($preview)}
				<div class="preview">
					{$page->getPreview()}
				</div>
			{/if}
		</div>
		*}
	{/foreach}
</div>
{else}
no pages.
{/if}
