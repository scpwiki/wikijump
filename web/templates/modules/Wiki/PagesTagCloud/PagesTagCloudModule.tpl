{if isset($tags)}
	<div class="pages-tag-cloud-box">
		{foreach from=$tags item=tag}
			<a class="tag" href="{$href}{$tag.tag|escape:'url'}{if isset($category)}/category/{$category->getName()|escape}{/if}{*#pages*}"
				style="font-size: {$tag.size}; color: rgb({$tag.color.r}, {$tag.color.g}, {$tag.color.b});"
				>{$tag.tag|escape}</a>
		{/foreach}
	</div>
{else}
	<p>
		{t}It seems you have no tags attached to pages. To attach a tag simply click on the <em>tags</em>
		button at the bottom of any page.{/t}
	</p>
{/if}
