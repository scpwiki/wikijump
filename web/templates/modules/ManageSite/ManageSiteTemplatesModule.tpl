<h1>Templates</h1>

<p>
	Templates are a convenient way to keep you Site Content well organized and consistent.
	Below you can assign default page skeletons/templates (pages from the <em>template:</em> category)
	to new created pages in particular categories.
</p>
{if isset($noTemplates)}
<p>
	Sorry, there are no templates available in this site.
	To create a template, simply create a new page named e.g. <br/>
	template:<em>new-template-name</em>.
</p>
{else}
	<table class="form">
		<tr>
			<td>
				<select name="category" size="10" id="sm-template-cats">
					{foreach from=$categories item=category}
						<option value="{$category->getCategoryId()}" style="padding: 0 2em" {if $category->getName()=="_default"}selected="selected"{/if}>{$category->getName()|escape}</option>
					{/foreach}
				</select>
			</td>
			<td>
				<select name="theme" id="sm-templates-list" size="10">
					<option value=""  style="padding: 0 1em">no default template</option>
					{foreach from=$templates item=template}
						<option value="{$template->getPageId()}"  style="padding: 0 1em">{$template->getTitle()|escape}</option>
					{/foreach}
				</select>
			</td>
		</tr>
	</table>

	<div class="buttons">
		<input type="button" value="cancel" id="sm-templates-cancel"/>
		<input type="button" value="save changes" id="sm-templates-save"/>
	</div>

	<div id="sm-template-preview">
		<h1>Template preview:</h1>
		{foreach from=$templates item=template}
			<div id="sm-template-preview-{$template->getPageId()}">
				<pre>{$template->getSource()|escape}</pre>
		{/foreach}
	</div>
{/if}
