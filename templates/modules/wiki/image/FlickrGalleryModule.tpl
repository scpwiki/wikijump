{if !$contentOnly}
<div class="flickr-gallery-box {if $makeHoverTitles}makeHoverTitles{/if}">
	<div>
{/if}
	{pager jsfunction="WIKIDOT.modules.FlickrGalleryModules.listeners.loadPage(event,#)" total=$pagerData.total_pages known=$pagerData.known_pages current=$pagerData.current_page}
	{if !$photos}
		Sorry, no photos.
	{else}	
		<div class="gallery-box">
			{foreach from=$photos item=photo}
				<div class="gallery-item {$size}">
					<table style="margin:0;padding:0"><tr><td  style="margin:0;padding:0">
						<a  href="{$photo.href}" {if !$disableBrowsing}onclick="WIKIDOT.modules.FlickrGalleryModules.listeners.showPhoto(event, '{$photo.id}');"{/if}><img title="{$photo.title|escape}" src="{$photo.src}" alt="{$photo.title}"/></a>
					</td></tr></table>
				</div>
			{/foreach}
		</div>
		<ul style="display: none">
			{foreach from=$photos item=photo}
				<li class="flickr-gallery-order">{$photo.id}</li>
			{/foreach}
		</ul>
	{/if}
{if !$contentOnly}
	</div>
	{if $parameters}
		<ul style="display: none">
			{foreach from=$parameters key=key item=value}
				<li class="flickr-gallery-parameter">{$key} ::: {$value|escape}</li>
			{/foreach}
		</ul>
		
	{/if}
</div>
{/if}

