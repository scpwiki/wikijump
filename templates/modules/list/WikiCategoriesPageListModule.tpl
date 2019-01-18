<ul>
	{foreach from=$pages item=page}
		<li>
			<a href="/{$page->getUnixName()}">{$page->getTitleOrUnixName()|escape}</a>
		</li>
	{/foreach}
</ul>