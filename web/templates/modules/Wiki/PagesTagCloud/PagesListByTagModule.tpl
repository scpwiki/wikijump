<a name="pages"></a>
<h2>{t 1=$tag}List of pages tagged with <u>%1</u>{/t}{if isset($category)} {t}from category{/t} <u>{$category->getName()}</u>{/if}:</h2>
{if isset($category)}
	<span style="float: right">(<a href="/{$pageUnixName}/tag/{$tag|escape:'url'}">{t}show from all categories{/t}</a>)</span>
{/if}

{if isset($pages)}

<div class="pages-list" id="tagged-pages-list">
	{foreach from=$pages item=page}
		<div class="pages-list-item">
			<div class="title">
				<a href="/{$page->getUnixName()}">{$page->getTitleOrUnixName()}</a>
			</div>
		</div>
	{/foreach}
</div>

{else}
	<p>
		{t}Somehow no pages have been found...{/t}
	</p>
{/if}
