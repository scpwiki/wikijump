{if isset($merged)}
	<table class="form">
		<tr>
			<th>
				{t}Page{/t}
			</th>
			<th>
				{t}Try to fix?{/t}
			</th>
		</tr>
		{foreach from=$merged item=page}
			<tr>
				<td>
					<a target="_blank" href="/{$page->getUnixName()}">{$page->getTitleOrUnixName()} ({$page->getUnixName()})</a>
				</td>
				<td>
					<input type="checkbox" class="checkbox" id="rename-dep-fix-{$page->getPageId()}"/>
				</td>
			</tr>
		{/foreach}
		<tr>
			<td colspan="2" style="text-align: right">
				<a href="javascript:;" onclick="Wikijump.modules.RenamePageModule.listeners.selectAll(event)">{t}select all{/t}</a>
				|  <a href="javascript:;" onclick="Wikijump.modules.RenamePageModule.listeners.unselectAll(event)">{t}unselect all{/t}</a>
			</td>
		</tr>
	</table>

	<p>
		{t}Select the pages you want to automaticaly fix dependencies for. If successful - the
		fixed pages will contain links to the renamed page. If not - they will contain broken links.{/t}
	</p>
{else}
	{t}No pages directly link or include this page.{/t}
{/if}
