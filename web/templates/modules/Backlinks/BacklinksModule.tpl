{ltext lang="en"}
	<h1>Other pages that depend on this page</h1>
	<h2>Backlinks</h2>
{/ltext}
{ltext lang="pl"}
	<h1>Inne strony zależne od tej</h1>
	<h2>Linki zwrotne</h2>
{/ltext}
{if isset($pages)}

<ul>
	{foreach from=$pages item=page}
		<li>
			<a href="/{$page->getUnixName()}">{$page->getTitleOrUnixName()} ({$page->getUnixName()})</a>
		</li>
	{/foreach}
</ul>


{else}
	{ltext lang="en"}
		No pages directly link to this page.
	{/ltext}
	{ltext lang="pl"}
		Żadne strony nie linkują bezpośrenio do tej.
	{/ltext}
{/if}
{ltext lang="en"}
	<h2>Inclusions</h2>
{/ltext}
{ltext lang="pl"}
	<h2>Włączenia  (poprzez <tt>[[include]]</tt>)</h2>
{/ltext}
{if isset($pagesI)}

<ul>
	{foreach from=$pagesI item=page}
		<li>
			<a href="/{$page->getUnixName()}">{$page->getTitleOrUnixName()} ({$page->getUnixName()})</a>
		</li>
	{/foreach}
</ul>


{else}
	{ltext lang="en"}
		No pages directly include this page.
	{/ltext}
	{ltext lang="pl"}
		Żadne strony bezpośrednio nie włączają tej strony.
	{/ltext}

{/if}

{if $pages || $pagesI}
	<p>
		{ltext lang="en"}
			Title of each page is given and page name (address) in parenthesis.
		{/ltext}
		{ltext lang="pl"}
			Powyżej podany jest tytuł każdej ze stron oraz ich nazwa (adres) w nawiasach.
		{/ltext}
	</p>
{/if}
