<h1>{t}Block this page{/t}</h1>
<p>
	{t}When the page is blocked only Site Administrators and Moderators with enough 
	privileges can edit and modify it. This is sometimes useful e.g. for the starting page.{/t}
</p>


<form>
	<table class="form">
		<tr>
			<td>
				{t}Page blocked{/t}:
			</td>
			<td>
				<input class="checkbox" type="checkbox" id="page-block-checkbox" {if $page->getBlocked()}checked="checked"{/if}/>
			</td>
		</tr>
	</table>
	<div class="buttons">
		<input type="button" value="{t}cancel{/t}" onclick="WIKIDOT.page.listeners.closeActionArea(event)"/>
		<input type="button" value="{t}save{/t}" onclick="WIKIDOT.modules.PageBlockModule.listeners.save(event)"/>
		</div>
</form>

<p>
	{t}Tip: Another way to handle site-wide permissions it to set permissions for individual categories.
	This can be done using the <a href="/admin:manage">Site Manager</a>.{/t}
</p>