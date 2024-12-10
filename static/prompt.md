# System
You are a helpful assistant specialising in data analysis in a PostgreSQL database.
Answer the questions by providing raw SQL code that is compatible with the PostgreSQL.
Do not format it as markdown (skip ```sql ``` blocks).

# User 
Here is a database schema:
```sql
 CREATE TABLE public._sqlx_migrations (checksum bytea NOT NULL, description text NOT NULL, execution_time bigint NOT NULL, installed_on timestamp with time zone NOT NULL, success boolean NOT NULL, version bigint NOT NULL);
 CREATE TABLE public.customer (created_at timestamp with time zone NOT NULL, firstname VARCHAR(255) NOT NULL, is_firstname_original boolean NOT NULL, telegram_id bigint NOT NULL);
 CREATE TABLE public.customer_telegram_channels_log (action USER-DEFINED NOT NULL, created_at timestamp with time zone NOT NULL, customer_telegram_id bigint NOT NULL, id integer NOT NULL, telegram_channel_username VARCHAR(255) NOT NULL, FOREIGN KEY (customer_telegram_id) REFERENCES public.customer(telegram_id), FOREIGN KEY (telegram_channel_username) REFERENCES public.telegram_channel(username));
 CREATE TABLE public.customer_tts_provider_log (created_at timestamp with time zone NOT NULL, customer_telegram_id bigint NOT NULL, text_to_speech_model smallint NOT NULL, FOREIGN KEY (customer_telegram_id) REFERENCES public.customer(telegram_id));
 CREATE TABLE public.podcast (audio_path VARCHAR(255) NOT NULL, created_at timestamp with time zone NOT NULL, customer_id bigint NOT NULL, id integer NOT NULL, text_path VARCHAR(255) NOT NULL, FOREIGN KEY (customer_id) REFERENCES public.customer(telegram_id));
 CREATE TABLE public.podcast_details (algorithm VARCHAR(255) NOT NULL, days_amount integer NOT NULL, duration_secs integer NOT NULL, llm_model VARCHAR(255) NOT NULL, news_amount integer NOT NULL, podcast_id integer NOT NULL, tokens_amount integer NOT NULL, tts_model VARCHAR(255) NOT NULL, FOREIGN KEY (podcast_id) REFERENCES public.podcast(id));
 CREATE TABLE public.podcast_feedback (podcast_id integer NOT NULL, rating integer NOT NULL, FOREIGN KEY (podcast_id) REFERENCES public.podcast(id));
 CREATE TABLE public.promocode (created_at timestamp with time zone NOT NULL, discount_amount real NOT NULL, expiration_date timestamp with time zone NOT NULL, id integer NOT NULL, usage_limit integer);
 CREATE TABLE public.promocode_usage (created_at timestamp with time zone NOT NULL, customer_telegram_id bigint NOT NULL, id integer NOT NULL, promocode_id integer NOT NULL, subscription_id integer NOT NULL, FOREIGN KEY (customer_telegram_id) REFERENCES public.customer(telegram_id), FOREIGN KEY (promocode_id) REFERENCES public.promocode(id), FOREIGN KEY (subscription_id) REFERENCES public.subscription(id));
 CREATE TABLE public.schedule (created_at timestamp with time zone NOT NULL, event jsonb NOT NULL, id integer NOT NULL, received boolean NOT NULL, scheduled_at timestamp with time zone NOT NULL);
 CREATE TABLE public.stripe_payment (created_at timestamp with time zone NOT NULL, customer_telegram_id bigint NOT NULL, id integer NOT NULL, status smallint NOT NULL, updated_at timestamp with time zone NOT NULL, FOREIGN KEY (customer_telegram_id) REFERENCES public.customer(telegram_id));
 CREATE TABLE public.subscription (created_at timestamp with time zone NOT NULL, customer_telegram_id bigint, end_date timestamp with time zone NOT NULL, id integer NOT NULL, start_date timestamp with time zone NOT NULL, updated_at timestamp with time zone NOT NULL, FOREIGN KEY (customer_telegram_id) REFERENCES public.customer(telegram_id));
 CREATE TABLE public.telegram_channel (title VARCHAR(255), username VARCHAR(255) NOT NULL);
 ```

And descripiton for some tables:
{{table_context}}


# User
{{user_request}}

# Assistant
