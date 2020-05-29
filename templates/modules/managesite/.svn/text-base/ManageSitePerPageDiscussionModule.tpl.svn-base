<h1>Per page discussion</h1>

<p>
	Do you want to enable "per page" forum threads? If so, every page from marked categories 
	would contain a button at the bottom called "discuss" which leads to a forum thread 
	dedicated to the discussion about the particular page.
</p>
<p>
	Also note that page discussion/comments can be enabled independently
	(and embedded within a page) for each page using the Comments module.
	More information in <a href="{$URL_DOCS}">documentation</a>.
</p>
<form>
	<div id="sm-pp-categories">
		<table class="form grid">
			{foreach from=$categories item=category}
				<tr>
					<td>
						{$category->getName()|escape}
					</td>
					<td>
						{if $category->getName() != '_default'}
							<input class="radio" type="radio" name="category-disscussion-{$category->getCategoryId()}" {if $category->getPerPageDiscussion() === null}checked="checked"{/if}/> default
						{/if}
					</td>
					<td>
						<input class="radio" type="radio" id="cat234-{$category->getCategoryId()}-e" name="category-disscussion-{$category->getCategoryId()}" {if $category->getPerPageDiscussion() === true}checked="checked"{/if}/> enable 
					</td>
					<td>
						<input class="radio" type="radio" id="cat234-{$category->getCategoryId()}-d" name="category-disscussion-{$category->getCategoryId()}" {if $category->getPerPageDiscussion() === false}checked="checked"{/if}/> disable
					</td>
				</tr>
			{/foreach}
		</table>
	</div>
	
	<div class="buttons">
		<input type="button" value="cancel" onclick="WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-welcome')"/>
		<input type="button" value="save" onclick="WIKIDOT.modules.ManageSitePerPageDiscussionModule.listeners.save(event)"/>
	</div>
</form>