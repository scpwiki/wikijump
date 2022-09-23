{**
* Prints a nice box with error message(s)
*}
{defmacro name="printErrorMessages"}
	{if $messages != null}
	<div style="text-align: center">
	<table class="errormess">
		<tr>
			<td>
				<img src="{$ui->image("warningsymbol.png")}" alt="*"/>
			</td>
			<td class="messholder">
				{foreach from=$messages item="message"}
					{$message}
				{/foreach}
			</td>
		</tr>
	</table>
	</div>
	{/if}
{/defmacro}

{**
* Prints a nice box with success message(s)
*}
{defmacro name="printSuccessMessages"}
	{if $messages != null}
	<div style="text-align: center">
	<table class="successmess">
		<tr>
			<td>
				<img src="{$ui->image("warning1.png")}" alt="*"/>
			</td>
			<td class="messholder">
		{foreach from=$messages item="message"}
			{$message}
		{/foreach}
			</td>
		</tr>
	</table>
	</div>
	{/if}
{/defmacro}



{**
* Creates a page number navigation element.
* @param $pagerData
* @param $golink
* @param $style
*}
{defmacro name="pager"}

{assign var=currentPage value=$pagerData->getCurrentPage()}
{assign var=knownPages value=$pagerData->getKnownPages()}
{assign var=totalPages value=$pagerData->getTotalPages()}
{if $totalPages > 1 || $knownPages>1}
<table class="pager"
	{if $style}
		style="{$style}"
	{/if}
>
	<tr>
		<td>
			strona {$currentPage}
			{if $totalPages != null}
				z {$totalPages}
				{assign var=pages value=$totalPages}
			{else}
				{assign var=pages value=$knownPages}
			{/if}
		</td>

		{if $currentPage != 1}
			{assign var="topage" value=$currentPage-1}
			<td>
				<a href="{$golink->copy()->addParameter("page_number", $topage)}"
					><img alt="prev" src="{$ui->image("aleft.gif")}"/></a>
			</td>
		{/if}
		<td>
			{if $currentPage > 3}
				<a href="{$golink->copy()->addParameter("page_number", 1)}">1</a>
			{/if}
			{if $currentPage > 4}
				<a href="{$golink->copy()->addParameter("page_number", 2)}">2</a>
			{/if}


			{if $currentPage  == 6}
				<a href="{$golink->copy()->addParameter("page_number", 3)}">3</a>
			{/if}
			{if $currentPage  > 6}
				...
			{/if}

			{if $currentPage-2 >= 1}
				{assign var="topage" value=$currentPage-2}
				<a href="{$golink->copy()->addParameter("page_number", $topage)}">{$currentPage-2}</a>
			{/if}

			{if ($currentPage-1) >= 1}
				{assign var="topage" value=$currentPage-1}
				<a href="{$golink->copy()->addParameter("page_number", $topage)}">{$currentPage-1}</a>
			{/if}

			<a class="current" href="{$golink->copy()->addParameter("page_number", $currentPage)}">{$currentPage}</a>

			{if $currentPage+1 <= $pages}
				{assign var="topage" value=$currentPage+1}
				<a href="{$golink->copy()->addParameter("page_number", $topage)}">{$currentPage+1}</a>
			{/if}

			{if $currentPage+2 <= $pages}
				{assign var="topage" value=$currentPage+2}
				<a href="{$golink->copy()->addParameter("page_number", $topage)}">{$currentPage+2}</a>
			{/if}

			{if $currentPage  < $pages-5}
				...
			{/if}

			{if $currentPage  == $pages-5}
				{assign var="topage" value=$tpages-2}
				<a href="{$golink->copy()->addParameter("page_number", $topage)}">{$pages-2}</a>
			{/if}

			{if $currentPage < $pages-3}
				{assign var="topage" value=$pages-1}
				<a href="{$golink->copy()->addParameter("page_number", $topage)}">{$pages-1}</a>
			{/if}

			{if $knownPages != null}
				...
			{/if}

			{if $currentPage < $pages-2}
				<a href="{$golink->copy()->addParameter("page_number", $pages)}">{$pages}</a>
			{/if}
		</td>

		{if $currentPage != $pages}
			<td>
				{assign var="topage" value=$currentPage+1}
				<a href="{$golink->copy()->addParameter("page_number", $topage)}"
				><img alt="next" src="{$ui->image("aright.gif")}"/></a>
			</td>
		{/if}
	</tr>
</table>

{/if}
{/defmacro}

{defmacro name="printUser"}
	<a href="{$HTTP_SCHEMA}://{$URL_HOST}/user:info/{$user->slug}">{$user->username|escape}</a>
{/defmacro}

