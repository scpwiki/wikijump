<div class="owindow" style="width: 80%;">
	<div class="title">
		{t}Page moved{/t}
	</div>
	<div class="content">
		<h1>{t}The page has been moved but...{/t}</h1>

		<p>
			{t}... there are a few dependencies left.{/t}
		</p>

		{if isset($pages)}
			<h2>{t}Backlinks{/t}</h2>
			<ul>
				{foreach from=$pages item=page}
					<li>
						<a target="_blank" href="/{$page->getUnixName()}">{$page->getTitleOrUnixName()} ({$page->getUnixName()})</a>
					</li>
				{/foreach}
			</ul>
		{/if}

		{if isset($pagesI)}
			<h2>{t}Inclusions{/t}</h2>
			<ul>
				{foreach from=$pagesI item=page}
					<li>
						<a target="_blank" href="/{$page->getUnixName()}">{$page->getTitleOrUnixName()} ({$page->getUnixName()})</a>
					</li>
				{/foreach}
			</ul>
		{/if}
		<p>
			{t}These pages were not fixed because the fixer algorithm could not fix them, they were
			edit-locked by another user or you just have not checked them to be fixed.{/t}
		</p>
		<p>
			{t}If you think they should be fixed - please do it by manually editing them.{/t}
		</p>
		<div class="button-bar">
			<a href="javascript:;" onclick="window.location.href='/'+Wikijump.modules.RenamePageModule.vars.newName;">{t}close window{/t}</a>
		</div>
	</div>
</div>
