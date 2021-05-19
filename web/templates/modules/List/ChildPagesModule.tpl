{if isset($pages)}
	<div class="child-pages-block">
		<ul>
			{foreach from=$pages item=p}
				<li><a href="/{$p->getUnixName()|escape}">{$p->getTitleOrUnixName()|escape}</a></li>
			{/foreach}
		</ul>
	</div>
{/if}
