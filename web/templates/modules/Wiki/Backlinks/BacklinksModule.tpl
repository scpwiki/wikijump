<div class="backlinks-module-box">
	{if isset($pages)}
		<ul>
			{foreach from=$pages item=page}
				<li>
					<a href="/{$page->getUnixName()}">{$page->getTitleOrUnixName()}</a>
				</li>
			{/foreach}
		</ul>
	{/if}
</div>
