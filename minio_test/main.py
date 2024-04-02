from minio import Minio, Object, error
import os

# Use minio==6 for this example

def _main():
    bucket_name = os.environ.get("S3_BUCKET")
    minio = Minio(
        endpoint=os.environ.get("S3_HOSTNAME"),
        access_key=os.environ.get("S3_ACCESSKEY"),
        secret_key=os.environ.get("S3_SECRETKEY"),
        secure=False,
    )
    assert minio.bucket_exists(bucket_name)
    for obj in minio.list_objects(bucket_name):
        print(obj.object_name)


if __name__ == "__main__":
    _main()
