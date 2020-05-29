<div class="search-box">

	
	<div class="query-area">
		<form action="javascript:;" id="search-form-user">
			<div>
				
				<input class="text" type="text" size="30" name="query" value="{$query|escape}"/>
				<input class="button" type="submit" value="{t}search{/t}"/>
			</div>		
		</form>
	</div>
	{* {$area} *}
	{* {$query_debug|escape} *}
	
	{if $errorMessage}
		<p>{$errorMessage}</p>
	{/if}
	
	{if $mode=='email'}
		
		{if $user}
		
			{assign var=profile value=$user->getProfile()}
			<div class="search-user-results">
				<div class="item">
					<div class="screen-name">
						{printuser user=$user image="true"}
						{if $profile->getRealName()!=''} (real name: {$profile->getRealName()|escape}){/if}
					</div>
					{if $profile->getAbout() != ''}
						<div class="about">
							{$profile->getAbout()|escape}
						</div>
					{/if}
				</div>
			</div>
		
		{else}
			<p>
				{t}Sorry, no user with such an email address.{/t}
			</p>
		{/if}
	
	{else}

		
		
		{capture name="destUrl"}/search:users/q/{$queryEncoded}/p/%d{/capture}
		{pager url=$smarty.capture.destUrl total=$pagerData.total_pages known=$pagerData.known_pages current=$pagerData.current_page}
		
		<div class="search-user-results">
			{if $users}
				{foreach from=$users item=user}
					{assign var=profile value=$user->getProfile()}
					<div class="item">
						<div class="screen-name">
							{printuser user=$user image="true"}
							{if $profile->getRealName()!=''} (real name: {$profile->getRealName()|escape}){/if}
						</div>
						{if $profile->getAbout() != ''}
							<div class="about">
								{$profile->getAbout()|escape}
							</div>
						{/if}
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
	{/if}

</div>