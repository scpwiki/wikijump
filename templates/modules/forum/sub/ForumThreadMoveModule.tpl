<h2>{t}Move thread{/t}</h2>
<p>
	{t}Just move this thread to a different location i.e. different category. This 
	should be also useful to "delete" threads - move to a hidden category somewhere else.{/t}
</p>

<form>
	<table class="form">
		<tr>
			<td>
				{t}Current category{/t}:
			</td>
			<td>
				{$category->getName()|escape}
			</td>
		</tr>
		<tr>
			<td>
				{t}New category{/t}:
			</td>
			<td>
				<select id="move-thread-category">
					{foreach from=$categories item=cat}
						<option value="{$cat.category->getCategoryId()}">{$cat.group->getName()|escape}: {$cat.category->getName()|escape}</option>
					{/foreach}
				</select>
			</td>
		</tr>
	</table>
	<div class="buttons">
		<input type="button" value="{t}cancel{/t}" onclick="$('thread-action-area').style.display='none'"/>
		<input type="button" value="{t}save{/t}" onclick="WIKIDOT.modules.ForumThreadMoveModule.listeners.move(event)"/>
	</div>
</form>