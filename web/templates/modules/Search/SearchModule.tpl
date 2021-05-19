<div class="search-box">

	<div class="query-area">
		<form action="javascript:;" id="search-form">
			<div>
				{if !$mini}
					{ltext lang="en"}
						Search query:
					{/ltext}
					{ltext lang="pl"}
						Szukana fraza:
					{/ltext}
				{/if}
				<input class="text" type="text" size="30" name="query" value="{$query|escape}"/>
				<input class="button" type="submit" value="{t}search{/t}"/>
			</div>
			{if !$mini}
			<div>
				<input class="radio" type="radio" name="area" value="" {if $area==null}checked="checked"{/if}/>{t}pages and forum{/t}
				<input class="radio" type="radio" name="area" value="p" {if $area=='p'}checked="checked"{/if}/>{t}pages only{/t}
				<input class="radio" type="radio" name="area" value="f" {if $area=='f'}checked="checked"{/if}/>{t}forum only{/t}
			</div>
			{/if}

		</form>
	</div>
	{* {$area} *}
	{* {$query_debug|escape} *}

	{if isset($message)}
		<p>{$message}</p>
	{/if}

	{capture name="destUrl"}/search:site{if isset($area)}/a/{$area}{/if}/q/{$queryEncoded}/p/%d{/capture}

	{pager url=$smarty.capture.destUrl total=$pagerData.total_pages known=$pagerData.known_pages current=$pagerData.current_page}

	<div class="search-results">
		{if isset($results)}
			{foreach from=$results item=result}
				<div class="item">
					<div class="title">
						{*<a href="{$result.url}/highlight/{$encodedQuery}">{$result.headline_title}</a>*}
						<a href="{$result.url}">{$result.headline_title}</a>
					</div>
					<div class="preview">
						{$result.headline_text}
					</div>
					<div class="url">
						{*<a href="{$result.url}">*}{$domain}{$result.url}{*</a>*}
					</div>
				</div>
			{/foreach}
		{else}
			{if $query && $query !=''}
				{t}Sorry, no results found for your query.{/t}
			{/if}
		{/if}
	</div>

	{if $countResults>7}
		{pager url=$smarty.capture.destUrl total=$pagerData.total_pages known=$pagerData.known_pages current=$pagerData.current_page}

	{/if}

</div>
{*
<p style="font-weight: bold">
	The search functionality has been temporarily disabled due to database performance issues.
</p>
*}
