<div class="search-box">

	<div class="query-area">
		<form action="dummy" id="search-form-all">
			<div>
				{t}Search query{/t}:
				<input class="text" type="text" size="30" name="query" id="search-form-all-input" value="{$query|escape}"/>
				<input class="button" type="submit" value="{t}search{/t}"/>
			</div>
			<div style="font-size: 87%; margin-top:5px;">
				<input id="search-all-pf" class="radio" type="radio" name="area" value="pf" {if !$area || $area=='pf'}checked="checked"{/if}/><label for="search-all-pf">{t}pages and forums{/t}</label>
				<input id="search-all-p" class="radio" type="radio" name="area" value="p" {if $area=='p'}checked="checked"{/if}/><label for="search-all-p">{t}pages only{/t}</label>
				<input id="search-all-f" class="radio" type="radio" name="area" value="f" {if $area=='f'}checked="checked"{/if}/><label for="search-all-f">{t}forums only{/t}</label>
			</div>


		</form>
	</div>
	{* {$area} *}
	{* {$query_debug|escape} *}


	{if isset($message)}
		<p>{$message}</p>
	{/if}

	{capture name="destUrl"}/search:all{if isset($area)}/a/{$area}{/if}/q/{$queryEncoded}/p/%d{/capture}
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
					<div class="site">
						site: <a href="{$HTTP_SCHEMA}://{$result.site->getDomain()|escape}">{$result.site->getName()|escape}</a>
					</div>
					<div class="url">
						{*<a href="{$result.url}">*}{$result.url}{*</a>*}
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
