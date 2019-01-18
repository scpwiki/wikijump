{if $nophoto}
<div style="error-block">
	This image does not seem to be available from Flickr.com
</div>
{else}

<div style="margin: 1em; text-align: center">
	<img src="{$src}" alt="flickr image"/>
</div>

	<table class="form">
		<tr>
			<td>
				Image size:
			</td>
			<td>
				<select class="select" id="wd-ed-imagewizard-size">
					<option value="square">square - 75x75 pixels</option>
					<option value="thumbnail">thumbnail - 100 on longest side</option>
					<option value="small">small - 240 on longest side</option>
					<option value="" selected="selected">medium - 500 on longest side</option>
					<option value="large">large - 1024 on longest side</option>
					<option value="original">original image</option>
				</select>
			</td>
		</tr>
	</div>
	
{/if}