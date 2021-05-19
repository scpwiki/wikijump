<h1>List of wanted pages</h1>

<p>
	Here is a list of pages that do not exist but there are links that point to them.
</p>

{if isset($res)}
<table class="form grid" style="margin: 1em auto;">
	<tr>
		<th>
			Linked from
		</th>
		<th>
			Linked to (wanted page name)
		</th>
	</tr>
	{foreach from=$res item=r key=name}
		<tr>
			<td>
				{foreach from=$r item=page}
					<a href="/{$page->getUnixName()}">{$page->getTitleOrUnixName()|escape}</a><br/>
				{/foreach}
			</td>
			<td>
				<a href="/{$name|escape}" class="newpage">{$name|escape}</a>
			</td>
		</tr>
	{/foreach}
</table>

{else}
	<p>
		No broken links found.
	</p>
{/if}
