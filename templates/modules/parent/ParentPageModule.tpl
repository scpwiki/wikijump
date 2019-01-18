<h1>{t}Parent page &amp; breadcrumbs navigation{/t}</h1>

<p>
	{t}Want to create cool breadcrumbs navigation? Create structured site layout?
	Choose the parent page (one-level-above) for this one.{/t}
</p>
<p>
	{t}If you do not want <a href="http://en.wikipedia.org/wiki/Breadcrumb_%28navigation%29" 
	target="_blank">breadcrumbs navigation</a> for this page - just leave the field below blank.{/t}
</p>


<div id="parent-set-error" class="error-block" style="display: none"></div>


<form onsubmit="WIKIDOT.modules.ParentPageModule.listeners.setParent(event)">
	<table class="form">
		<tr>
			<td>
				{t}Parent page name{/t}:
			</td>
			<td>
				<div class="autocomplete-container" style="width: 20em">
					<input type="text" id="parent-page-name" class="autocomplete-input text" name="parentPageName" size="35" value="{$parentPageName|escape}"/>
					<div id="parent-page-name-list" class="autocomplete-list"></div>
				</div>
			</td>
		</tr>
	</table>
	<div class="buttons">
		<input type="button" value="{t}cancel{/t}" onclick="WIKIDOT.page.listeners.closeActionArea(event)"/>
		<input type="button" value="{t}clear parent{/t}" onclick="$('parent-page-name').value=''; return false;"/>
		<input type="submit" value="{t}save parent page{/t}"/>
	</div>
</form>
