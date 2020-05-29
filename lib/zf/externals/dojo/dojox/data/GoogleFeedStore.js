dojo.provide("dojox.data.GoogleFeedStore");
dojo.experimental("dojox.data.GoogleFeedStore");

dojo.require("dojox.data.GoogleSearchStore");

dojo.declare("dojox.data.GoogleFeedStore", dojox.data.GoogleSearchStore,{
	// summary:
	//	A data store for retrieving RSS and Atom feeds from Google. The 
	//  feeds can come from any source, which is specified in the "url"
	//  parameter of the query passed to the "fetch" function.
	//	The following attributes are supported on each item:
	//		<ul>
	//			<li>title - The feed entry title.</li>
	//			<li>link - The URL for the HTML version of the feed entry.</li>
	//			<li>content - The full content of the blog post, in HTML format</li>
	//			<li>summary - A snippet of information about the feed entry, in plain text</li>
	//			<li>published - The string date on which the entry was published.
	//				You can parse the date with new Date(store.getValue(item, "published")</li>
	//			<li>categories - An array of string tags for the entry</li>
	//		</ul>
	//	The query accepts one parameter: url - The URL of the feed to retrieve
	_type: "",
	_googleUrl: "http://ajax.googleapis.com/ajax/services/feed/load",
	_attributes: ["title", "link", "author", "published",
					"content", "summary", "categories"],
	_queryAttr: "url",

	_processItem: function(item, request) {
		this.inherited(arguments);
		item["summary"] = item["contentSnippet"];
		item["published"] = item["publishedDate"];
	},

	_getItems: function(data){
		return data['feed'] && data.feed[['entries']] ? data.feed[['entries']] : null;
	},

	_createContent: function(query, callback, request){
		var cb = this.inherited(arguments);
		cb.num = (request.count || 10) + (request.start || 0);
		return cb;
	}
});