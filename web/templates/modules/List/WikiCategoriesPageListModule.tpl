<ul>
	{foreach from=$pages item=page}
		<li>
			<a href="/{$page->slug}">{$page->title|escape}</a>
		</li>
	{/foreach}
</ul>
