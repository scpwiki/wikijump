<div class="top-rated-pages-box">

	<div class="top-rated-pages-list">
		{foreach from=$pages item=page}
			<div class="list-item">
				<a href="/{$page->getUnixName()}">{$page->getTitleOrUnixName()|escape}</a>
				<span style="color: #777">({t}rating{/t}: {$page->getRate()}{if $page->getTemp("numberComments")!==null}, {t}comments{/t}: {$page->getTemp("numberComments")}{/if})</span>
			</div>
		{/foreach}
	</div>

</div>