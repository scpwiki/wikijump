<div class="owindow photowindow" id="photowindow">
	<div class="close"><a href="javascript:;" onclick="OZONE.dialog.cleanAll()" style="text-decoration: none;">X</a></div>
	<div class="phototitle">
		{$photo.title|escape}
	</div>
	<table style="margin: 0 auto; width: 100%;">
		<tr>
			<td>
				<a id="photo-nav-prev" href="javascript:;" onclick="WIKIDOT.modules.FlickrGalleryModules.listeners.showPreviousPhoto(event)">«</a>
			</td>
			<td>
				<img class="flickrphoto" src="{$photoSrc}" width="{$dimensions.width}" height="{$dimensions.height}"/>
			</td>
			<td>
				<a id="photo-nav-next" href="javascript:;" onclick="WIKIDOT.modules.FlickrGalleryModules.listeners.showNextPhoto(event)">»</a>
			</td>
		</tr>
	</table>
	
	{if $photo.description}
		<div class="description">
			{$photo.description |escape}
		</div>
	{/if}
	
	<div class="hostedby">
		This image is hosted by <strong><a href="http://flickr.com">Flickr</a></strong>.
		Click <a href="{$photoUrl}">here</a> for the original image.
	</div>
</div><!-- end -->