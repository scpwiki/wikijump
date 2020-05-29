import org.apache.lucene.analysis.standard.StandardAnalyzer;
import org.apache.lucene.document.Document;
import org.apache.lucene.queryParser.QueryParser;
import org.apache.lucene.search.Hits;
import org.apache.lucene.search.IndexSearcher;
import org.apache.lucene.search.Query;

public class WikidotSearcher {
	public static void main(String[] args) {
		
		if (args.length != 2) {
			System.err.println("Usage:");
			System.err.println("  java -jar lucene_search.jar <index_dir> <search_query>");
			return;
		}
		
		try {
			String indexPath = args[0];
			String queryString = args[1];

			IndexSearcher indexSearcher = new IndexSearcher(indexPath);
			QueryParser qp = new QueryParser("content", new StandardAnalyzer());
			Query query = qp.parse(queryString);

			Hits hits = indexSearcher.search(query);
			int hitCount = hits.length();
			
			for (int i = 0; i < hitCount; i++) {
				Document doc = hits.doc(i);
				System.out.println(doc.get("fts_id"));
			}
			
		} catch (Exception e) {
			System.err.println("Exception: " + e.getMessage());
			e.printStackTrace();
		}
	}
}
