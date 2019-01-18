<div class="sites-tag-cloud-box">
	{foreach from=$tags item=tag}
		<a class="tag" href="{$href}{$tag.tag|escape:'url'}"
			style="font-size: {$tag.size}%; color: rgb({$tag.color.r}, {$tag.color.g}, {$tag.color.b});"
			>{$tag.tag|escape}</a>
	{/foreach}
</div>