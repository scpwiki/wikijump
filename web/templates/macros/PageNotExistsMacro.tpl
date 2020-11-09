
{defmacro name="pageNotExistsMacro"}

<p>
{t 1=$wikiPage escape=no}The page <em>%1</em> you want to access does not exist.{/t}
</p>
<ul>
	<li><a href="javascript:;" onclick="Wikijump.page.listeners.editClick(event)">{t}create page{/t}</a></li>
</ul>

{/defmacro}
