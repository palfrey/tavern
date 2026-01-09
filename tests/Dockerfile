FROM python:3.11.14-alpine3.23
WORKDIR /work
RUN apk add --no-cache bash
RUN wget https://raw.githubusercontent.com/vishnubob/wait-for-it/9995b721327eac7a88f0dce314ea074d5169634f/wait-for-it.sh && \
    chmod +x wait-for-it.sh && mv wait-for-it.sh /bin/wait-for-it
ADD requirements.txt /work/
RUN pip install -r requirements.txt
ENV PYTHONUNBUFFERED=1
ADD . /work
CMD pytest -vvv -s .
