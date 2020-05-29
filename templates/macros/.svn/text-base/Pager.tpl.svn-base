{**
* Creates a page number navigation element.
* @param $pagerData
* @param $gofunction
*}
{defmacro name="pager"}

{assign var=currentPage value=$pagerData.current_page}
{assign var=knownPages value=$pagerData.known_pages}
{assign var=totalPages value=$pagerData.total_pages}
{if $totalPages > 1 || $knownPages>1}
<div class="pager">
	{if $totalPages > 1 || $knownPages>1}

		page {$currentPage}
		{if $totalPages != null}
			of {$totalPages}
			{assign var=pages value=$totalPages}
		{else}
			{assign var=pages value=$knownPages}
		{/if} 	
			
		{if $currentPage != 1}
			{assign var="topage" value=$currentPage-1}
				<a href="javascript:;" onclick="{$gofunction($topage)}"
						>&laquo;</a>
				</td>
		{/if}
	
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
		
			
		{if $currentPage != $pages}
			{assign var="topage" value=$currentPage+1}
			<a href="{$golink->copy()->addParameter("page_number", $topage)}"
			>&raquo;</a>
			
		{/if}

	{/if}
</div>
{/defmacro}