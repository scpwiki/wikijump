{foreach from=$categories item=category}
	<div>
		<h3>{$category->getName()|escape}</h3>
		<a id="category-pages-toggler-{$category->getCategoryId()}" href="javascript:;" onclick="WIKIDOT.modules.WikiCategoriesModule.listeners.toggleListPages(event, {$category->getCategoryId()})">+ list pages</a>
		<div style="display: none" id="category-pages-{$category->getCategoryId()}"></div>
	</div>
{/foreach}

