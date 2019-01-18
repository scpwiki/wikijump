<div class="forum-recent-posts-box" >
	<form onsubmit="return false;" action="dummy.html" method="get">
		<table class="form">
			<tr>
				<td>
					{t}From categories{/t}: 
				</td>
				<td>
					<select id="recent-posts-category">
						<option value="" selected="selected">{t}All categories{/t}</option>
						{foreach from=$cats item=cat}
							<option value="{$cat.category->getCategoryId()}">{$cat.group->getName()|escape}: {$cat.category->getName()|escape}</option>
						{/foreach}
					</select>
					<input class="button" type="button" value="{t}update{/t}" onclick="WIKIDOT.modules.ForumRecentPostsModule.listeners.updateList()"/>
				</td>
			</tr>
		</table>
	</form>

	<div id="forum-recent-posts-list">
		{module name="forum/ForumRecentPostsListModule"}
	</div>

</div>