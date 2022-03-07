<div class="backlinks-module-box">
	{if $pages}
		<ul>
			{foreach from=$pages item=page}
				<li>
					<a href="/{$page->slug}">{$page->title}</a>
				</li>
			{/foreach}
		</ul>
	{/if}
</div>
